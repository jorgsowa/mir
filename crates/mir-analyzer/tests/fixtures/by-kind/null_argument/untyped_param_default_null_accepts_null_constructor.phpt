===description===
An untyped param defaulting to null is implicitly nullable even when its docblock says non-null.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Lib {
    /** @param string $file */
    public function __construct($file = null) {}
}
new Lib(null);
