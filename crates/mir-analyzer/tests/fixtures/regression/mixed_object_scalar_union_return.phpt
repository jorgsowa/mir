===description===
G5: a `string|Cat` value satisfies a declared `string|Animal` return type when
`Cat extends Animal`. The object atom resolves through the inheritance graph while
the scalar atom is checked structurally — the mixed union must not be a false positive.
===config===
suppress=UnusedParam
===file===
<?php
class Animal {}
class Cat extends Animal {}
class Box {
    private bool $flag;
    private Cat $cat;
    public function __construct(bool $flag, Cat $cat) {
        $this->flag = $flag;
        $this->cat = $cat;
    }
    public function get(): string|Animal {
        if ($this->flag) {
            return "name";
        }
        return $this->cat;
    }
}
===expect===
