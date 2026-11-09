===description===
A conditional `@param` type picks its branch from the other arguments, or their defaults, before the argument is checked.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class P {
    /** @param ($flags is 1 ? (callable(int): string) : (callable(string): string)) $cb */
    public static function f(callable $cb, int $flags = 0): string { return ''; }
}
P::f(static fn (string $m): string => $m);
P::f(static fn (int $m): string => (string) $m, 1);
P::f(static fn (int $m): string => (string) $m);
//   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $cb of f() expects 'callable whose parameter #1 accepts string', got 'callable whose parameter #1 only accepts int'
