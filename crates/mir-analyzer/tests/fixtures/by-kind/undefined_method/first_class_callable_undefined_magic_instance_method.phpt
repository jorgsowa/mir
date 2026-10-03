===description===
FirstClassCallable:UndefinedMagicInstanceMethod
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Test {
    public function __call(string $name, array $args): mixed {
        return match ($name) {
            default => throw new Error("Undefined method"),
        };
    }
}
$test = new Test();
$closure = $test->length(...);
$length = $closure();

===expect===
