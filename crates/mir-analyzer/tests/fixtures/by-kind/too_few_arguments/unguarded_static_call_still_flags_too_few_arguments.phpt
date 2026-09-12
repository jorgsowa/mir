===description===
Without an existence guard, the same static call is checked normally and
TooFewArguments is reported.
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
class NewApi {
    public static function run(int $a, int $b): void {}
}

function check(): void {
    NewApi::run(1);
}
===expect===
TooFewArguments@7:4-7:18: Too few arguments for run(): expected 2, got 1
