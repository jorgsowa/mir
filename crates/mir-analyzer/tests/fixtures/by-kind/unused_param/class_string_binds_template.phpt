===description===
class-string<T> argument binds T to the named class, not the bound
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @template T */
class Wrapper {}

class Foo {}
class Bar {}

/**
 * @template T of object
 * @param class-string<T> $cls
 * @return Wrapper<T>
 */
function make(string $cls): Wrapper { return new Wrapper(); }
//            ^^^^^^^^^^^ UnusedParam: Parameter $cls is never used

$fooWrapper = make(Foo::class);
$barWrapper = make(Bar::class);
/** @mir-check $fooWrapper is Wrapper<Foo> */
/** @mir-check $barWrapper is Wrapper<Bar> */
