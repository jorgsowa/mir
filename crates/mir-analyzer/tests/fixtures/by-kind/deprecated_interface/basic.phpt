===description===
DeprecatedInterface fires when a class implements a deprecated interface.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @deprecated use NewLogger instead */
interface OldLogger {
    public function log(string $msg): void;
}

class FileLogger implements OldLogger {
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ DeprecatedInterface: Interface OldLogger is deprecated: use NewLogger instead
    public function log(string $msg): void {}
}
