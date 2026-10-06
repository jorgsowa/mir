===description===
No ImplicitToStringCast when class defines __toString — PHP coerces implicitly, no warning needed
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class A {
    public function __toString(): string
    {
        return "hello";
    }
}

/** @mutation-free */
function fooFoo(string $b): void {}
fooFoo(new A());
