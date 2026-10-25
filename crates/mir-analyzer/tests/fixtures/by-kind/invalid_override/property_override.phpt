===description===
From PHP 8.5 `#[Override]` on a property needs a non-private parent property (class or interface), including promoted and trait properties.
===file===
<?php
interface HasName {
    public string $name { get; }
}

class Base {
    public int $id = 0;
    private int $secret = 0;
}

final class Child extends Base implements HasName {
    #[Override]
    public int $id = 1;

    #[Override]
    public string $name = '';

    #[Override]
    public int $secret = 0;
//  ^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::$secret has #[\Override] attribute, but no matching parent property exists

    #[Override]
    public int $extra = 0;
//  ^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::$extra has #[\Override] attribute, but no matching parent property exists

    public function __construct(#[Override] public int $promoted = 0) {}
//                                          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidOverride: Child::$promoted has #[\Override] attribute, but no matching parent property exists
}

trait Tagged {
    #[Override]
    public string $tag = '';
}

final class UsesTrait {
    use Tagged;
//      ^^^^^^^ InvalidOverride: UsesTrait::$tag has #[\Override] attribute, but no matching parent property exists
}
