===description===
`@property T $value` on a generic interface substitutes the receiver's own
concrete type argument, instead of leaking the raw unbound template.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingConstructor errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/**
 * @template T
 * @property T $value
 */
interface Box {}

/** @param Box<int> $b */
function test(Box $b): void {
    /** @mir-check $b->value is int */
    $_ = 1;
}
