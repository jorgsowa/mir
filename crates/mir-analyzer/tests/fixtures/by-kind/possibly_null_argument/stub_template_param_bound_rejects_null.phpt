===description===
A stub class's template param whose bound excludes null (`T of object`)
rejects a possibly-null constructor argument even though bound violations
aren't reported for stub classes. Method templates (WeakReference::create) and
user templates keep reporting the bound violation alone.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
interface Container {
    public function get(string $id): ?object;
}

function stubs(Container $c, ?string $class): void {
    new ReflectionClass($c->get('a'));
//                      ^^^^^^^^^^^^ PossiblyNullArgument: Argument $objectOrClass of ReflectionClass::__construct() might be null
    new ReflectionClass($class);
//                      ^^^^^^ PossiblyNullArgument: Argument $objectOrClass of ReflectionClass::__construct() might be null
    WeakReference::create($c->get('b'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'TIn' inferred as 'object|null' does not satisfy bound 'object'
    $o = $c->get('c');
    if ($o !== null) {
        new ReflectionClass($o);
        WeakReference::create($o);
    }
}

/** @template T of object */
final class Holder {
    /** @param T $value */
    public function __construct(object $value) {}
}

function user(Container $c): void {
    new Holder($c->get('d'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^ InvalidTemplateParam: Template type 'T' inferred as 'object|null' does not satisfy bound 'object'
}
