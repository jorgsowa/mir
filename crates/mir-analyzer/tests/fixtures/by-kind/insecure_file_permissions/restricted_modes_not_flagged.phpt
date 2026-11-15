===description===
Modes without the others-write bit, masks that remove it, computed modes and
`mkdir()`, whose mode the umask restricts, aren't reported.
===file===
<?php
function publish(string $file, string $dir, int $mode): void {
    chmod($file, 0644);
    chmod($file, 0775);
    chmod($file, 0o600);
    chmod($file, $mode);
    umask(022);
    umask(0007);
    mkdir($dir, 0777, true);
}
