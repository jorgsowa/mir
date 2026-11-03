===description===
A static call from outside the class references it even if `new self` is the only instantiation
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class Factory {
    public static function make(): self {
        return new self();
    }
}

Factory::make();
