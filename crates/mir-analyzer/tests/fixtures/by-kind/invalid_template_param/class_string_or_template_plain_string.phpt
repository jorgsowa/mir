===description===
A plain `string` argument for a `class-string<T>|T` param may name a class, so
it fills the class-string alternative and leaves `T` unbound instead of binding
`T` to `string` and violating `T of object`. A stub class (ReflectionClass)
leaves a bound-violating binding unbound rather than reporting it on every
later call; a user class still reports it.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T of object */
final class Mirror {
    /** @param class-string<T>|T $target */
    public function __construct($target) {}

    /** @return T */
    public function make(): object { throw new LogicException(); }
}

/**
 * @template T of object
 * @param class-string<T>|T $target
 * @return T
 */
function build($target): object { throw new LogicException(); }

/** @param non-empty-string $name */
function plainString(string $class, string $name): void {
    $m = new Mirror($class);
    /** @mir-check $m is Mirror */
    $m->make();
    $n = new Mirror($name);
    /** @mir-check $n is Mirror */
    $built = build($class);
    /** @mir-check $built is object */
}

function classLike(object $o): void {
    $m = new Mirror(stdClass::class);
    /** @mir-check $m is Mirror<stdClass> */
    $o2 = new Mirror($o);
    /** @mir-check $o2 is Mirror<object> */
}

function reflection(string $class, ?object $maybe, int $i): void {
    $r = new ReflectionClass($class);
    /** @mir-check $r is ReflectionClass */
    $r->getName();
    $lower = new ReflectionClass('stdClass');
    /** @mir-check $lower is ReflectionClass<stdClass> */
    $r2 = new ReflectionClass($maybe);
//                            ^^^^^^ PossiblyNullArgument: Argument $objectOrClass of ReflectionClass::__construct() might be null
    $r2->getName();
}

function userClassStillReported(int $i): void {
    new Mirror($i);
//  ^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'int' does not satisfy bound 'object'
}
