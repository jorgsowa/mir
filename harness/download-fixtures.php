#!/usr/bin/env php
<?php

declare(strict_types=1);

$here = __DIR__;
$fixturesDir = $here . '/fixtures';

if (!is_dir($fixturesDir) && !mkdir($fixturesDir, 0777, true) && !is_dir($fixturesDir)) {
    fwrite(STDERR, "Failed to create $fixturesDir\n");
    exit(1);
}

foreach (require $here . '/packages.php' as ['slug' => $slug, 'url' => $url, 'tag' => $tag]) {
    $dest = $fixturesDir . '/' . $slug;
    if (is_dir($dest)) {
        echo "== $slug: fixture already exists at $dest — skipping clone.\n";
    } else {
        echo "== $slug: cloning $url @ $tag ...\n";
        passthru(sprintf(
            'git clone --depth=1 --branch %s %s %s',
            escapeshellarg($tag),
            escapeshellarg($url),
            escapeshellarg($dest)
        ), $status);
        if ($status !== 0) {
            fwrite(STDERR, "== $slug: git clone failed\n");
            exit(1);
        }
    }

    echo "== $slug: composer install (--no-dev) ...\n";
    passthru(sprintf(
        'composer install --working-dir=%s --no-dev --no-scripts --no-plugins --no-interaction --prefer-dist --ignore-platform-reqs --quiet',
        escapeshellarg($dest)
    ), $status);
    if ($status !== 0) {
        fwrite(STDERR, "== $slug: composer install failed\n");
        exit(1);
    }
}

echo "\nDone. Run the baseline harness with:\n  harness/run.php\n";
