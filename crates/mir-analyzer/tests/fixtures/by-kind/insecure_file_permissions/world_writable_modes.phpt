===description===
A literal `chmod()` mode with the others-write bit, or a `umask()` that doesn't
remove it, leaves files writable by every user on the system.
===file===
<?php
function publish(string $file, string $dir): void {
    chmod($file, 0777);
//  ^^^^^^^^^^^^^^^^^^ InsecureFilePermissions: Insecure file permissions: chmod() mode 0777 lets every user on the system write the file
    chmod($dir . '/data.json', 0666);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InsecureFilePermissions: Insecure file permissions: chmod() mode 0666 lets every user on the system write the file
    umask(0);
//  ^^^^^^^^ InsecureFilePermissions: Insecure file permissions: umask(0000) lets every user on the system write the files and directories created after it
}
