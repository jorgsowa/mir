===description===
`use A { A::foo as bar; }` where `foo` is a real method on `A` must not be
flagged UndefinedTraitAliasMethod.
===config===
<mir>
  <issueHandlers>
    <UnusedMethod errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
trait A {
    public function foo(): void {}
}

class C {
    use A {
        A::foo as bar;
    }
}
