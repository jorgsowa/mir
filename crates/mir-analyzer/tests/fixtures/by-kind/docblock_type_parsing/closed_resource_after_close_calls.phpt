===description===
fclose(), pclose() and closedir() close the variable they are given; a closed handle still fits `resource`, and a conditional close leaves it a resource.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param resource $handle */
function inspect($handle): void {}

function files(string $path, bool $done): void {
    $file = fopen($path, 'r');
    if ($file === false) {
        return;
    }
    if ($done) {
        fclose($file);
    }
    /** @mir-check $file is resource */
    inspect($file);
    fclose($file);
    /** @mir-check $file is closed-resource */
    inspect($file);
}

function others(string $path): void {
    $dir = opendir($path);
    if ($dir !== false) {
        closedir($dir);
        /** @mir-check $dir is closed-resource */
        echo 'closed';
    }
    $pipe = popen('ls', 'r');
    if ($pipe !== false) {
        pclose($pipe);
        /** @mir-check $pipe is closed-resource */
        echo 'closed';
    }
}
