===description===
Implicit nullability from a null default applies to functions, instance methods and static methods.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param string $f */
function plain($f = null): void {}

class Lib {
    /** @param string $f */
    public function inst($f = null): void {}
    /** @param string $f */
    public static function stat($f = null): void {}
}
plain(null);
(new Lib())->inst(null);
Lib::stat(null);
===expect===
