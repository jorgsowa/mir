===description===
A `preg_match` guard reads its pattern from a class or global constant.
===config===
<mir>
  <issueHandlers>
    <MixedArrayAccess errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
const DIGITS = '/^\d+$/';

class Q {
    const ID = '/^\d+$/';

    public function check(): void {
        $x = (string) $_GET['x'];
        if (preg_match(self::ID, $x)) {
            echo $x;
        }
        if (!preg_match(DIGITS, $x)) {
            return;
        }
        echo $x;
    }
}
