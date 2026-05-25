===description===
impureGlobalImmutable
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
ImpureGlobalVariable
===ignore===
TODO
