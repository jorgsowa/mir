===description===
`class-string<T>` nested in an array/list param binds T from the element and
does not report the argument as invalid.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}
class Bar {}

/**
 * @template T
 * @param array<class-string<T>> $classes
 * @return T
 */
function fromArray(array $classes) { throw new Exception(); }

/**
 * @template T
 * @param list<class-string<T>> $classes
 * @return T
 */
function fromList(array $classes) { throw new Exception(); }

/**
 * @template T
 * @param array<string, class-string<T>> $classes
 * @return T
 */
function fromMap(array $classes) { throw new Exception(); }

function test(): void {
    $a = fromArray([Foo::class]);
    /** @mir-check $a is Foo */
    echo get_class($a);

    $b = fromList([Foo::class, Bar::class]);
    /** @mir-check $b is Foo|Bar */
    echo get_class($b);

    $c = fromMap(['k' => Foo::class]);
    /** @mir-check $c is Foo */
    echo get_class($c);
}
===expect===
