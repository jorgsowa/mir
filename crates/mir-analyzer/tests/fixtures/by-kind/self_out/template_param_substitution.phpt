===description===
@psalm-self-out substitutes a method-level @template bound from the call's
argument, same as a normal @return type would.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @template T
 */
class Box {
    /**
     * @template U
     * @param U $v
     * @psalm-self-out Box<U>
     */
    public function set($v): void {}
}

$box = new Box();
$box->set("hello");
/** @mir-check $box is Box<string> */

===expect===
