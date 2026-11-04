===description===
A static method's own `@template T` is inferred from the call, not taken from a same-named class template bound by `@extends`
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {}
class B {}

/** @template T of object */
abstract class Base {
    /**
     * @template T of object
     * @param T $value
     * @return list<T>
     */
    public static function wrap(object $value): array {
        return [$value];
    }
}

/** @extends Base<B> */
final class Child extends Base {
    public function f(): void {
        $wrapped = self::wrap(new A());
//                 ^^^^^^^^^^^^^^^^^^^ ShadowedTemplateParam: Method template parameter 'T' shadows class-level template parameter with the same name
        /** @mir-check $wrapped is list<A> */
        echo 1;
    }
}
