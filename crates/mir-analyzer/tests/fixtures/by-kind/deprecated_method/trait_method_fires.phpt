===description===
DeprecatedMethod fires when calling a deprecated method that comes from a used trait.
===config===
suppress=UnusedParam
===file===
<?php
trait Logger {
    /** @deprecated use log() instead */
    public function write(): void {}
}
class App {
    use Logger;
}

function test(App $app): void {
    $app->write();
}
===expect===
DeprecatedMethod@11:4-11:17: Method App::write() is deprecated: use log() instead
