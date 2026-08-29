#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mir_analyzer::composer::Psr4Map;
use mir_analyzer::db::{Fqcn, MirDatabase};
use mir_analyzer::{AnalysisSession, IndexCancel, IndexParallelism, PhpVersion};
use mir_types::Name;

pub struct FullSymfonyFixture {
    pub root: PathBuf,
    pub session: AnalysisSession,
    pub request: Arc<str>,
    pub request_stack: Arc<str>,
    pub parameter_bag: Arc<str>,
    pub input_bag: Arc<str>,
    pub uri_signer: Arc<str>,
    pub string_functions: Arc<str>,
    pub ascii_slugger: Arc<str>,
    pub session_interface: Arc<str>,
    pub compiled_url_matcher_trait: Arc<str>,
    pub truncate_mode: Arc<str>,
    pub router: Arc<str>,
}

pub fn fixture_root() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("MIR_SYMFONY_FIXTURE") {
        let root = PathBuf::from(path);
        if is_valid_fixture_root(&root) {
            return Some(root);
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("benches/fixtures/symfony");
    is_valid_fixture_root(&root).then_some(root)
}

fn is_valid_fixture_root(root: &Path) -> bool {
    root.join("composer.json").is_file()
        && root
            .join("src/Symfony/Component/HttpFoundation/Request.php")
            .is_file()
        && root
            .join("src/Symfony/Component/String/Resources/functions.php")
            .is_file()
}

fn arc_path(path: PathBuf) -> Arc<str> {
    Arc::from(path.to_string_lossy().as_ref())
}

fn batch_entry(path: &Arc<str>) -> (Arc<str>, Arc<str>) {
    let text = std::fs::read_to_string(path.as_ref()).unwrap();
    (path.clone(), Arc::from(text.as_str()))
}

fn chunked<T>(items: &[T], size: usize) -> impl Iterator<Item = &[T]> {
    items.chunks(size)
}

pub fn load_full_symfony_fixture() -> Option<FullSymfonyFixture> {
    let root = fixture_root()?;
    let psr4 = Psr4Map::from_composer(&root)
        .expect("full Symfony fixture must have a valid composer.json");

    let mut files = psr4.project_files();
    let vendor_files = psr4.all_vendor_files();
    files.extend(vendor_files);
    files.sort();
    files.dedup();

    let batch: Vec<(Arc<str>, Arc<str>)> = files
        .iter()
        .filter_map(|path| {
            let text = std::fs::read_to_string(path).ok()?;
            Some((
                Arc::from(path.to_string_lossy().as_ref()),
                Arc::from(text.as_str()),
            ))
        })
        .collect();

    let session = AnalysisSession::new(PhpVersion::LATEST).with_psr4(Arc::new(psr4));
    session.ensure_all_stubs();

    let cancel = IndexCancel::new();
    for chunk in chunked(&batch, 256) {
        session.index_batch(chunk, IndexParallelism::Sequential, &cancel);
    }
    session.finalize_index();

    let request = arc_path(root.join("src/Symfony/Component/HttpFoundation/Request.php"));
    let request_stack =
        arc_path(root.join("src/Symfony/Component/HttpFoundation/RequestStack.php"));
    let parameter_bag =
        arc_path(root.join("src/Symfony/Component/HttpFoundation/ParameterBag.php"));
    let input_bag = arc_path(root.join("src/Symfony/Component/HttpFoundation/InputBag.php"));
    let uri_signer = arc_path(root.join("src/Symfony/Component/HttpFoundation/UriSigner.php"));
    let string_functions =
        arc_path(root.join("src/Symfony/Component/String/Resources/functions.php"));
    let ascii_slugger =
        arc_path(root.join("src/Symfony/Component/String/Slugger/AsciiSlugger.php"));
    let session_interface =
        arc_path(root.join("src/Symfony/Component/HttpFoundation/Session/SessionInterface.php"));
    let compiled_url_matcher_trait = arc_path(
        root.join("src/Symfony/Component/Routing/Matcher/Dumper/CompiledUrlMatcherTrait.php"),
    );
    let truncate_mode = arc_path(root.join("src/Symfony/Component/String/TruncateMode.php"));
    let router = arc_path(root.join("src/Symfony/Component/Routing/Router.php"));

    let fx = FullSymfonyFixture {
        root,
        session,
        request,
        request_stack,
        parameter_bag,
        input_bag,
        uri_signer,
        string_functions,
        ascii_slugger,
        session_interface,
        compiled_url_matcher_trait,
        truncate_mode,
        router,
    };

    let analysis_files = fx.analysis_target_files();
    analyze_fixture_files(&fx, &analysis_files);

    Some(fx)
}

pub fn analyze_fixture_files(fx: &FullSymfonyFixture, files: &[Arc<str>]) {
    let paths: Vec<PathBuf> = files.iter().map(|f| PathBuf::from(f.as_ref())).collect();
    let _ = fx
        .session
        .analyze_paths(&paths, &mir_analyzer::BatchOptions::new().without_symbols());
}

pub fn reanalyze_fixture_files(fx: &FullSymfonyFixture, files: &[Arc<str>]) {
    let cancel = IndexCancel::new();
    let _ = fx.session.reanalyze_files_cancellable(files, &cancel);
}

pub fn fixture_batch_entry(path: &Arc<str>) -> (Arc<str>, Arc<str>) {
    batch_entry(path)
}

pub fn fqcn<'db>(db: &'db dyn MirDatabase, name: &str) -> Fqcn<'db> {
    Fqcn::new(db, Name::new(name))
}
impl FullSymfonyFixture {
    pub fn analysis_target_files(&self) -> Vec<Arc<str>> {
        vec![
            self.request.clone(),
            self.request_stack.clone(),
            self.parameter_bag.clone(),
            self.input_bag.clone(),
            self.uri_signer.clone(),
            self.string_functions.clone(),
            self.ascii_slugger.clone(),
            self.session_interface.clone(),
            self.compiled_url_matcher_trait.clone(),
            self.truncate_mode.clone(),
            self.router.clone(),
        ]
    }
}
