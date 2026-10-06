===description===
Without an existence guard, the same static call is checked normally and
TooFewArguments is reported.
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
    NewApi::run(1);
//  ^^^^^^^^^^^^^^ TooFewArguments: Too few arguments for run(): expected 2, got 1
}
