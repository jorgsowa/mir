===description===
DeprecatedMethod fires when calling a deprecated method that comes from a used trait.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
//  ^^^^^^^^^^^^^ DeprecatedMethod: Method App::write() is deprecated: use log() instead
}
