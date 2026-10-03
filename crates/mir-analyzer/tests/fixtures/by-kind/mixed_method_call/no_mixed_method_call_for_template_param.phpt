===description===
G1: method call on an unconstrained template param must not emit MixedMethodCall —
T is an intentionally parameterised placeholder, not truly mixed.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingPropertyType errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 * @param T $obj
 */
function call_method($obj): void {
    // T is a template param — must not fire MixedMethodCall
    $obj->doSomething();
}

/**
 * @template T of object
 * @param T $obj
 */
function call_bounded_method($obj): void {
    // T of object — must not fire MixedMethodCall
    $obj->process();
}
===expect===
