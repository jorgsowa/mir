#!/usr/bin/env php
<?php

declare(strict_types=1);

// Usage: harness/run.php [--update]. MIR_BIN overrides the mir binary path.

$here = __DIR__;
$repoRoot = dirname($here);
$fixturesDir = $here . '/fixtures';
$baselinesDir = $here . '/baselines';

$update = in_array('--update', $argv, true);

$mirBin = getenv('MIR_BIN') ?: $repoRoot . '/target/release/mir';
if (!is_executable($mirBin)) {
    fwrite(STDERR, "== mir binary not found at $mirBin — building release ...\n");
    passthru('cd ' . escapeshellarg($repoRoot) . ' && cargo build --release -p mir-php', $buildStatus);
    if ($buildStatus !== 0) {
        exit(2);
    }
}

/**
 * @param list<string> $args
 * @return array{0: string, 1: string, 2: int} [stdout, stderr, exit code]
 */
function runMir(array $args, string $cwd): array
{
    $descriptors = [1 => ['pipe', 'w'], 2 => ['pipe', 'w']];
    $proc = proc_open($args, $descriptors, $pipes, $cwd);
    if (!is_resource($proc)) {
        fwrite(STDERR, 'failed to start: ' . implode(' ', $args) . "\n");
        exit(2);
    }
    $stdout = stream_get_contents($pipes[1]);
    $stderr = stream_get_contents($pipes[2]);
    fclose($pipes[1]);
    fclose($pipes[2]);
    $exitCode = proc_close($proc);
    return [$stdout, $stderr, $exitCode];
}

function countBaselineEntries(string $baselineFile): int
{
    if (!is_file($baselineFile)) {
        return 0;
    }
    $xml = simplexml_load_file($baselineFile);
    if ($xml === false) {
        return 0;
    }
    $count = 0;
    foreach ($xml->file as $file) {
        foreach ($file->children() as $kind) {
            $count += count($kind->code);
        }
    }
    return $count;
}

$overallStatus = 0;

foreach (require $here . '/packages.php' as ['slug' => $slug]) {
    $fixtureDir = $fixturesDir . '/' . $slug;
    $baselineFile = $baselinesDir . '/' . $slug . '.xml';

    echo "== $slug\n";

    if (!is_dir($fixtureDir)) {
        fwrite(STDERR, "  no fixture at $fixtureDir — run harness/download-fixtures.php first\n");
        $overallStatus = 1;
        continue;
    }

    if ($update) {
        [, $stderr, $code] = runMir(
            [$mirBin, 'src', '--set-baseline', $baselineFile, '--no-progress', '-q'],
            $fixtureDir
        );
        if ($code !== 0) {
            fwrite(STDERR, "  mir failed: $stderr\n");
            $overallStatus = 1;
            continue;
        }
        echo '  ' . countBaselineEntries($baselineFile) . " issue(s) baselined -> $baselineFile\n";
        continue;
    }

    [$stdout, $stderr] = runMir(
        [$mirBin, 'src', '--baseline', $baselineFile, '--format', 'json', '--no-progress', '-q'],
        $fixtureDir
    );

    $issues = json_decode($stdout, true);
    if (!is_array($issues)) {
        fwrite(STDERR, "  failed to parse mir output as JSON:\n$stdout\n$stderr\n");
        $overallStatus = 1;
        continue;
    }

    if (count($issues) > 0) {
        echo '  ' . count($issues) . " NEW issue(s) not in baseline:\n";
        foreach ($issues as $issue) {
            $kind = array_key_first($issue['kind']);
            $file = $issue['location']['file'];
            $snippet = $issue['snippet'] ?? '';
            $suffix = $snippet !== '' ? " ($snippet)" : '';
            echo "    $file: $kind$suffix\n";
        }
        $overallStatus = 1;
    } else {
        echo "  clean\n";
    }
}

exit($overallStatus);
