===description===
An interface's or abstract hooked property is a contract, and a virtual property (no hook touches its backing value) has no storage; neither needs a constructor.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
interface Named {
    public string $name { get; set; }
}

abstract class Base {
    abstract public int $size { get; }
}

class Label extends Base implements Named {
    public string $name {
        get => 'label';
        set => null;
    }
    public int $size { get => 1; }
}
