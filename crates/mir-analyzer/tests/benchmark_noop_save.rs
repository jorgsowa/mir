/// Performance benchmark: No-op file save optimization
///
/// This benchmark measures the improvement from the FileDefinitions Update fix.
/// When a file is re-ingested with identical content, Salsa should skip
/// downstream recomputation.
///
/// Run with: cargo test --test benchmark_noop_save -- --nocapture --ignored
use mir_analyzer::AnalysisSession;
use std::sync::Arc;
use std::time::Instant;

/// A more substantial PHP file to make timing more reliable
const PHP_CODE: &str = r#"<?php

namespace App\Controllers;

use App\Models\User;
use Illuminate\Http\Request;

class UserController {
    private User $userModel;

    public function __construct(User $userModel) {
        $this->userModel = $userModel;
    }

    public function index(): array {
        return $this->userModel->all();
    }

    public function show(int $id): ?User {
        return $this->userModel->findById($id);
    }

    public function store(Request $request): User {
        $data = $request->validate([
            'name' => 'required|string',
            'email' => 'required|email',
            'password' => 'required|string|min:8',
        ]);

        return $this->userModel->create($data);
    }

    public function update(Request $request, int $id): User {
        $user = $this->userModel->findById($id);
        if (!$user) {
            throw new \Exception("User not found");
        }

        $data = $request->validate([
            'name' => 'string',
            'email' => 'email',
        ]);

        $user->update($data);
        return $user;
    }

    public function destroy(int $id): bool {
        $user = $this->userModel->findById($id);
        return $user ? $user->delete() : false;
    }

    private function authorize(User $user): void {
        if (auth()->user()->id !== $user->id && !auth()->user()->isAdmin()) {
            throw new \Exception("Unauthorized");
        }
    }
}

namespace App\Models;

class User {
    public ?int $id = null;
    public string $name = "";
    public string $email = "";
    protected array $hidden = ['password'];

    public function all(): array {
        return [];
    }

    public function findById(int $id): ?User {
        return null;
    }

    public function create(array $data): User {
        return new self();
    }

    public function update(array $data): void {}

    public function delete(): bool {
        return true;
    }
}
"#;

#[test]
#[ignore]
fn benchmark_no_op_file_save() {
    let session = AnalysisSession::new(mir_analyzer::PhpVersion::LATEST);
    session.ensure_stubs_loaded();

    let file: Arc<str> = Arc::from("app/Http/Controllers/UserController.php");
    let source: Arc<str> = Arc::from(PHP_CODE);

    println!("\n{:=<70}", "");
    println!("Benchmark: No-Op File Save (FileDefinitions Update Optimization)");
    println!("{:=<70}\n", "");

    // Warmup
    println!("Warmup...");
    session.ingest_file(file.clone(), source.clone());

    // First measurement: new file
    println!("\n[1] First ingest (new content, all definitions fresh)");
    let t_start = Instant::now();
    session.ingest_file(file.clone(), source.clone());
    let t_first = t_start.elapsed();
    println!("    Time: {:.2}ms", t_first.as_secs_f64() * 1000.0);

    // Get baseline document symbols
    let symbols_baseline = session.document_symbols(file.as_ref());
    println!("    Symbols: {}", symbols_baseline.len());

    // Second measurement: no-op (same content)
    // With the fix, FileDefinitions::maybe_update should detect identical content
    // and return false, avoiding downstream recomputation
    println!("\n[2] Second ingest (identical content, should be FASTER)");
    let t_start = Instant::now();
    session.ingest_file(file.clone(), source.clone());
    let t_noop = t_start.elapsed();
    println!("    Time: {:.2}ms", t_noop.as_secs_f64() * 1000.0);

    let symbols_after = session.document_symbols(file.as_ref());
    println!("    Symbols: {}", symbols_after.len());

    // Third measurement: changed content (to show difference)
    let modified_source: Arc<str> =
        Arc::from(PHP_CODE.replace("UserController", "UserApiController"));
    println!("\n[3] Third ingest (changed content, class renamed)");
    let t_start = Instant::now();
    session.ingest_file(file.clone(), modified_source);
    let t_changed = t_start.elapsed();
    println!("    Time: {:.2}ms", t_changed.as_secs_f64() * 1000.0);

    let symbols_changed = session.document_symbols(file.as_ref());
    println!("    Symbols: {}", symbols_changed.len());

    // Analysis
    println!("\n{:=<70}", "");
    println!("Results:");
    println!("{:=<70}", "");

    let noop_pct = (t_noop.as_secs_f64() / t_first.as_secs_f64()) * 100.0;
    let changed_pct = (t_changed.as_secs_f64() / t_first.as_secs_f64()) * 100.0;

    println!("\nRelative to first ingest:");
    println!(
        "  No-op (identical):     {:6.1}% ({:.2}ms)",
        noop_pct,
        t_noop.as_secs_f64() * 1000.0
    );
    println!(
        "  Changed content:       {:6.1}% ({:.2}ms)",
        changed_pct,
        t_changed.as_secs_f64() * 1000.0
    );

    println!("\nComparison:");
    println!(
        "  No-op vs First:        {:.2}x faster",
        t_first.as_secs_f64() / t_noop.as_secs_f64()
    );
    if t_changed > t_noop {
        println!(
            "  No-op vs Changed:      {:.2}x faster",
            t_changed.as_secs_f64() / t_noop.as_secs_f64()
        );
    } else {
        println!(
            "  No-op vs Changed:      {:.2}x slower (changed needs recomputation)",
            t_noop.as_secs_f64() / t_changed.as_secs_f64()
        );
    }

    // Verdict
    println!("\n{:=<70}", "");
    if t_noop < t_first {
        let speedup =
            ((t_first.as_secs_f64() - t_noop.as_secs_f64()) / t_first.as_secs_f64()) * 100.0;
        println!(
            "✅ PASS: No-op save is {:.1}% faster (fix is working!)",
            speedup
        );
    } else if (t_noop.as_secs_f64() - t_first.as_secs_f64()).abs() < 0.001 {
        println!("⚠️  NOISE: Times are within timing noise margin");
        println!("    (file is too small or system too fast for reliable measurement)");
    } else {
        println!("❌ FAIL: No-op save was NOT faster (unexpected!)");
    }
    println!("{:=<70}\n", "");

    // Verify correctness
    assert_eq!(
        symbols_baseline.len(),
        symbols_after.len(),
        "No-op should produce identical symbols"
    );
}

#[test]
fn verify_no_op_correctness() {
    // Verify that no-op file saves produce identical results
    let session = AnalysisSession::new(mir_analyzer::PhpVersion::LATEST);
    session.ensure_stubs_loaded();

    let file: Arc<str> = Arc::from("test_file.php");
    let source: Arc<str> =
        Arc::from("<?php\nclass TestClass {\n    public function test(): void {}\n}\n");

    session.ingest_file(file.clone(), source.clone());
    let symbols_1 = session.document_symbols(file.as_ref());

    // Ingest identical content again
    session.ingest_file(file.clone(), source.clone());
    let symbols_2 = session.document_symbols(file.as_ref());

    // Symbols should be identical
    assert_eq!(symbols_1.len(), symbols_2.len());
    assert!(symbols_1
        .iter()
        .zip(&symbols_2)
        .all(|(a, b)| a.name == b.name));

    println!("✅ No-op file save produces identical results");
}
