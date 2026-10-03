===description===
Impure global immutable
===config===
<mir>
  <issueHandlers>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @immutable
 */
class A {
    /**
     * @global string $bar
     */
    public function foo() : string {
        global $bar;
        return $bar;
    }
}
===expect===
