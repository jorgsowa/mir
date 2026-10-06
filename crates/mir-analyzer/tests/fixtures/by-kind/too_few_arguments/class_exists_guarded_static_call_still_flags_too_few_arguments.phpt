===description===
A class_exists() guard on a resolvable class does NOT suppress
TooFewArguments for the static call in the guarded branch: the guard is
true in the snapshot's own environment, so the branch is live and the
snapshot signature is authoritative.
===config===
<mir>
  <issueHandlers>
    <MissingReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class NewApi {
    public static function run(int $a, int $b): void {}
}

function check(): void {
    if (class_exists('NewApi')) {
        NewApi::run(1);
//      ^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for run(): expected 2, got 1
    }
}
