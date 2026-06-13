===description===
DeprecatedMethod fires when calling a deprecated instance method.
===config===
suppress=UnusedParam
===file===
<?php
class Logger {
    /** @deprecated use log() instead */
    public function write(string $msg): void {}
}

function test(Logger $l): void {
    $l->write('hello');
}
===expect===
DeprecatedMethod@8:5-8:23: Method Logger::write() is deprecated: use log() instead
