===description===
$this->prop .= 'x' updates the property's flow-tracked type instead of leaving it stale.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public string $log = '';

    public function run(): void {
        $this->log = 'a';
        $this->log .= 'b';
        $x = $this->log;
        /** @mir-check $x is 'ab' */
        $_ = $x;
    }
}
===expect===
