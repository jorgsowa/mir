===description===
does not report read property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
    private string $name = 'bar';

    public function getName(): string {
        return $this->name;
    }
}
