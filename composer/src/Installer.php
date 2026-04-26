<?php

declare(strict_types=1);

namespace Mir\Composer;

use Composer\InstalledVersions;
use Composer\Script\Event;

/**
 * Downloads the prebuilt mir binary that matches the installed package version
 * and the host platform, verifies its sha256 checksum against the sidecar
 * published next to the release asset, and places it in the package's bin/
 * directory so that the composer/bin/mir shim can exec it.
 *
 * Trust model: integrity rests on the GitHub Releases asset and its `.sha256`
 * sidecar being uncompromised. The sidecar is fetched over the same TLS
 * channel as the archive, so it protects against transport corruption but not
 * against a release-pipeline compromise. If a stronger guarantee is required,
 * sigstore/cosign signing in the release workflow is the standard upgrade.
 */
final class Installer
{
    private const PACKAGE = 'mir-analyzer/mir';
    private const REPO = 'jorgsowa/mir';
    private const VERSION_MARKER = '.mir-version';

    public static function install(Event $event): void
    {
        $io = $event->getIO();

        $installPath = self::resolveInstallPath();
        if ($installPath === null) {
            // Running inside the source repo itself, or the package isn't
            // installed via composer — nothing to do.
            return;
        }

        $version = self::resolveVersion();
        if ($version === null) {
            $io->writeError('<warning>mir: could not determine package version, skipping binary download.</warning>');
            return;
        }

        try {
            [$os, $target] = self::detectPlatform();
        } catch (\RuntimeException $e) {
            $io->writeError('<warning>mir: ' . $e->getMessage() . '. Build from source instead.</warning>');
            return;
        }

        $binDir = $installPath . '/bin';
        $isWindows = $os === 'windows';
        $binaryName = $isWindows ? 'mir.exe' : 'mir';
        $binaryPath = $binDir . '/' . $binaryName;
        $markerPath = $binDir . '/' . self::VERSION_MARKER;

        // Skip download if the marker file confirms we already have this
        // exact version. The marker avoids executing the (potentially
        // tampered) existing binary just to read its --version.
        if (
            is_file($binaryPath)
            && is_file($markerPath)
            && trim((string) @file_get_contents($markerPath)) === $version
        ) {
            return;
        }

        $archiveExt = $isWindows ? 'zip' : 'tar.gz';
        $archiveName = "mir-{$target}.{$archiveExt}";
        $tag = "v{$version}";
        $base = sprintf('https://github.com/%s/releases/download/%s', self::REPO, $tag);
        $archiveUrl = "{$base}/{$archiveName}";
        $checksumUrl = "{$archiveUrl}.sha256";

        $io->write("<info>mir: downloading {$archiveName} for {$tag}...</info>");

        if (!is_dir($binDir) && !@mkdir($binDir, 0755, true) && !is_dir($binDir)) {
            throw new \RuntimeException("mir: failed to create {$binDir}");
        }

        $tmpArchive = self::tempPath('mir-archive-');
        $tmpExtractDir = self::makeTempDir();
        try {
            self::download($archiveUrl, $tmpArchive);

            $expected = self::parseChecksum(self::httpGet($checksumUrl));
            $actual = hash_file('sha256', $tmpArchive);
            if ($actual === false || !hash_equals($expected, $actual)) {
                throw new \RuntimeException(
                    "mir: checksum mismatch for {$archiveName}"
                    . " (expected {$expected}, got " . var_export($actual, true) . ')',
                );
            }

            $extractedBinary = $isWindows
                ? self::extractZipEntry($tmpArchive, $tmpExtractDir, $binaryName)
                : self::extractTarEntry($tmpArchive, $tmpExtractDir, $binaryName);

            self::assertContainedIn($extractedBinary, $tmpExtractDir);
            if (!is_file($extractedBinary) || is_link($extractedBinary)) {
                throw new \RuntimeException("mir: extracted entry is not a regular file: {$extractedBinary}");
            }

            // Atomic-ish replace: copy to a sibling tempfile in $binDir then rename.
            $stagingPath = $binaryPath . '.new';
            if (!@copy($extractedBinary, $stagingPath)) {
                throw new \RuntimeException("mir: failed to write {$stagingPath}");
            }
            if (!$isWindows) {
                @chmod($stagingPath, 0755);
            }
            if (!@rename($stagingPath, $binaryPath)) {
                @unlink($stagingPath);
                throw new \RuntimeException("mir: failed to install binary at {$binaryPath}");
            }

            if (@file_put_contents($markerPath, $version . "\n") === false) {
                $io->writeError('<warning>mir: failed to write version marker; binary will be re-downloaded next install.</warning>');
            }
        } finally {
            @unlink($tmpArchive);
            self::rmRecursive($tmpExtractDir);
        }

        $io->write("<info>mir: installed {$tag} to {$binaryPath}</info>");
    }

    private static function resolveInstallPath(): ?string
    {
        if (!class_exists(InstalledVersions::class)) {
            return null;
        }
        if (!InstalledVersions::isInstalled(self::PACKAGE)) {
            return null;
        }
        $path = InstalledVersions::getInstallPath(self::PACKAGE);
        return $path !== null ? rtrim($path, '/\\') : null;
    }

    private static function resolveVersion(): ?string
    {
        if (!class_exists(InstalledVersions::class) || !InstalledVersions::isInstalled(self::PACKAGE)) {
            return null;
        }
        $pretty = InstalledVersions::getPrettyVersion(self::PACKAGE);
        if ($pretty === null) {
            return null;
        }
        $pretty = ltrim($pretty, 'v');
        // Reject branch aliases like "dev-main" — there's no matching release.
        if (!preg_match('/^\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?$/', $pretty)) {
            return null;
        }
        return $pretty;
    }

    /**
     * @return array{0:string,1:string} [os, rust-target-triple]
     */
    private static function detectPlatform(): array
    {
        $os = match (PHP_OS_FAMILY) {
            'Linux' => 'linux',
            'Darwin' => 'darwin',
            'Windows' => 'windows',
            default => throw new \RuntimeException('unsupported OS: ' . PHP_OS_FAMILY),
        };

        $machine = strtolower(trim((string) php_uname('m')));
        $arch = match (true) {
            in_array($machine, ['x86_64', 'amd64', 'x64'], true) => 'x86_64',
            in_array($machine, ['aarch64', 'arm64'], true) => 'aarch64',
            default => throw new \RuntimeException("unsupported architecture: {$machine}"),
        };

        $target = match ([$os, $arch]) {
            ['linux', 'x86_64'] => 'x86_64-unknown-linux-gnu',
            ['linux', 'aarch64'] => 'aarch64-unknown-linux-gnu',
            ['darwin', 'x86_64'] => 'x86_64-apple-darwin',
            ['darwin', 'aarch64'] => 'aarch64-apple-darwin',
            ['windows', 'x86_64'] => 'x86_64-pc-windows-msvc',
            default => throw new \RuntimeException("no prebuilt binary for {$os}-{$arch}"),
        };

        return [$os, $target];
    }

    private static function download(string $url, string $dest): void
    {
        $fh = fopen($dest, 'wb');
        if ($fh === false) {
            throw new \RuntimeException("mir: cannot write to {$dest}");
        }
        try {
            $ctx = self::httpContext(60);
            $src = @fopen($url, 'rb', false, $ctx);
            if ($src === false) {
                throw new \RuntimeException("mir: failed to download {$url}");
            }
            try {
                if (stream_copy_to_stream($src, $fh) === false) {
                    throw new \RuntimeException("mir: download stream failed for {$url}");
                }
            } finally {
                fclose($src);
            }
        } finally {
            fclose($fh);
        }
    }

    private static function httpGet(string $url): string
    {
        $data = @file_get_contents($url, false, self::httpContext(30));
        if ($data === false) {
            throw new \RuntimeException("mir: failed to fetch {$url}");
        }
        return $data;
    }

    /** @return resource */
    private static function httpContext(int $timeout)
    {
        return stream_context_create([
            'http' => [
                'follow_location' => 1,
                'max_redirects' => 5,
                'header' => "User-Agent: mir-composer-installer\r\n",
                'timeout' => $timeout,
                'ignore_errors' => false,
            ],
            'ssl' => [
                'verify_peer' => true,
                'verify_peer_name' => true,
            ],
        ]);
    }

    private static function parseChecksum(string $content): string
    {
        $first = strtok($content, "\n");
        if ($first === false) {
            throw new \RuntimeException('mir: empty checksum file');
        }
        $hash = strtolower(trim((string) strtok($first, " \t")));
        if (!preg_match('/^[0-9a-f]{64}$/', $hash)) {
            throw new \RuntimeException('mir: malformed sha256 checksum');
        }
        return $hash;
    }

    /**
     * Validate that a relative archive entry name does not attempt path
     * traversal, absolute paths, or NUL injection. Applied to every entry
     * before extraction.
     */
    private static function isSafeEntryName(string $name): bool
    {
        if ($name === '' || strlen($name) > 1024) {
            return false;
        }
        if (str_contains($name, "\0")) {
            return false;
        }
        // No absolute paths (POSIX or Windows-style).
        if ($name[0] === '/' || $name[0] === '\\') {
            return false;
        }
        if (preg_match('/^[A-Za-z]:[\\\\\/]/', $name)) {
            return false;
        }
        // No `..` segments.
        $normalized = str_replace('\\', '/', $name);
        foreach (explode('/', $normalized) as $segment) {
            if ($segment === '..') {
                return false;
            }
        }
        return true;
    }

    /**
     * Extract a single named entry from a tar.gz archive into $destDir, after
     * validating every entry's name. Returns the absolute path to the
     * extracted file inside $destDir.
     */
    private static function extractTarEntry(string $archive, string $destDir, string $entryName): string
    {
        if (!class_exists(\PharData::class)) {
            throw new \RuntimeException('mir: ext-phar is required to extract tar.gz archives');
        }

        // Phar normalizes paths to their realpath in the stream URL (e.g. on
        // macOS `/tmp` becomes `/private/tmp`), so resolve before computing
        // the prefix used to derive relative entry names.
        $archiveReal = realpath($archive);
        if ($archiveReal === false) {
            throw new \RuntimeException("mir: archive disappeared before extraction: {$archive}");
        }
        $phar = new \PharData($archiveReal);
        $found = false;
        $pharPrefix = 'phar://' . $archiveReal . '/';
        $iterator = new \RecursiveIteratorIterator($phar, \RecursiveIteratorIterator::SELF_FIRST);
        foreach ($iterator as $path => $file) {
            /** @var \PharFileInfo $file */
            $full = (string) $path;
            $relative = str_starts_with($full, $pharPrefix)
                ? substr($full, strlen($pharPrefix))
                : $full;
            if (!self::isSafeEntryName($relative)) {
                throw new \RuntimeException("mir: rejected unsafe archive entry: {$relative}");
            }
            if ($file->isLink()) {
                throw new \RuntimeException("mir: rejected symlink archive entry: {$relative}");
            }
            if (!$file->isFile() && !$file->isDir()) {
                throw new \RuntimeException("mir: rejected non-regular archive entry: {$relative}");
            }
            if ($relative === $entryName) {
                $found = true;
            }
        }
        if (!$found) {
            throw new \RuntimeException("mir: archive does not contain {$entryName}");
        }
        $phar->extractTo($destDir, $entryName, true);
        unset($phar);

        return $destDir . '/' . $entryName;
    }

    /**
     * Extract a single named entry from a zip archive into $destDir, after
     * validating every entry's name. Returns the absolute path to the
     * extracted file.
     */
    private static function extractZipEntry(string $archive, string $destDir, string $entryName): string
    {
        if (!class_exists(\ZipArchive::class)) {
            throw new \RuntimeException('mir: ext-zip is required to extract Windows archives');
        }

        $zip = new \ZipArchive();
        if ($zip->open($archive) !== true) {
            throw new \RuntimeException("mir: failed to open zip {$archive}");
        }
        try {
            $found = false;
            for ($i = 0; $i < $zip->numFiles; $i++) {
                $name = $zip->getNameIndex($i);
                if ($name === false || !self::isSafeEntryName($name)) {
                    throw new \RuntimeException('mir: rejected unsafe archive entry: ' . var_export($name, true));
                }
                if ($name === $entryName) {
                    $found = true;
                }
            }
            if (!$found) {
                throw new \RuntimeException("mir: archive does not contain {$entryName}");
            }
            if ($zip->extractTo($destDir, [$entryName]) !== true) {
                throw new \RuntimeException('mir: failed to extract zip entry');
            }
        } finally {
            $zip->close();
        }

        return $destDir . '/' . $entryName;
    }

    /**
     * Confirm that $candidate, after symlink resolution, is contained inside
     * $parent. Defends against extractors that might honor symlinks pointing
     * outside the staging directory.
     */
    private static function assertContainedIn(string $candidate, string $parent): void
    {
        $realParent = realpath($parent);
        if ($realParent === false) {
            throw new \RuntimeException("mir: staging directory disappeared: {$parent}");
        }
        $candidateDir = dirname($candidate);
        $realCandidateDir = realpath($candidateDir);
        if ($realCandidateDir === false) {
            throw new \RuntimeException("mir: extracted file's directory missing: {$candidate}");
        }
        $sep = DIRECTORY_SEPARATOR;
        $needle = rtrim($realParent, $sep) . $sep;
        if (
            $realCandidateDir !== $realParent
            && !str_starts_with($realCandidateDir . $sep, $needle)
        ) {
            throw new \RuntimeException("mir: extracted path escaped staging dir: {$candidate}");
        }
    }

    private static function tempPath(string $prefix): string
    {
        $path = tempnam(sys_get_temp_dir(), $prefix);
        if ($path === false) {
            throw new \RuntimeException('mir: failed to create temporary file');
        }
        return $path;
    }

    private static function makeTempDir(): string
    {
        $base = sys_get_temp_dir();
        for ($i = 0; $i < 8; $i++) {
            $candidate = $base . DIRECTORY_SEPARATOR . 'mir-extract-' . bin2hex(random_bytes(8));
            if (@mkdir($candidate, 0700)) {
                return $candidate;
            }
        }
        throw new \RuntimeException('mir: failed to create temporary directory');
    }

    private static function rmRecursive(string $path): void
    {
        if (!is_dir($path) || is_link($path)) {
            @unlink($path);
            return;
        }
        $entries = @scandir($path);
        if ($entries !== false) {
            foreach ($entries as $entry) {
                if ($entry === '.' || $entry === '..') {
                    continue;
                }
                self::rmRecursive($path . DIRECTORY_SEPARATOR . $entry);
            }
        }
        @rmdir($path);
    }
}
