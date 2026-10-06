===description===
One param
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface I {
    /**
     * @param array $i
     */
    public function foo(array $i) : void;
}

class C implements I {
    public function foo(array $c) : void {
        return;
    }
}
