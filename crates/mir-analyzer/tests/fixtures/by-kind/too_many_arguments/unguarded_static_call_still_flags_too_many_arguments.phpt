===description===
Without an existence guard, the same static call is checked normally and
TooManyArguments is reported.
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
    public static function run(int $a): void {}
}

function check(): void {
    NewApi::run(1, 2);
//                 ^ TooManyArguments: Too many arguments for run(): expected 1, got 2
}
