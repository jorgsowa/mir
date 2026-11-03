===description===
A read of a property the winning declaration of a duplicated class lacks still counts
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file:a.php===
<?php
final class Provider {
    private bool $loaded = false;

    public function load(): bool {
        if (!$this->loaded) {
//                  ^^^^^^ UndefinedProperty: Property Provider::$loaded does not exist
            $this->loaded = true;
//          ^^^^^^^^^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property Provider::$loaded is deprecated
            return true;
        }
        return false;
    }
}
===file:b.php===
<?php
final class Provider {
    public function load(): bool { return true; }
}
(new Provider())->load();
