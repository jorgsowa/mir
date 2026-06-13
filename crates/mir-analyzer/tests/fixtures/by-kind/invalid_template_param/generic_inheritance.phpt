===description===
Subclasses at various inheritance levels satisfy template bounds - inheritance-aware checking
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
class Base {}
class Child extends Base {}
class GrandChild extends Child {}

/**
 * @template T of Base
 */
class Container {
    /**
     * @param T $_item
     */
    public function add($_item): void {}
}

function testTemplateInheritance() {
    $container = new Container();
    $container->add(new Base());
    $container->add(new Child());
    $container->add(new GrandChild());
}

interface Taggable {
    public function tag(): string;
}

class TaggedBase implements Taggable {
    public function tag(): string { return 'base'; }
}

class TaggedChild extends TaggedBase {}

/**
 * @template T of Taggable
 */
class TagContainer {
    /**
     * @param T $_item
     */
    public function add($_item): void {}
}

function testInterfaceInheritance() {
    $container = new TagContainer();
    $container->add(new TaggedBase());
    $container->add(new TaggedChild());
}

class Level1 {}
class Level2 extends Level1 {}
class Level3 extends Level2 {}
class Level4 extends Level3 {}

/**
 * @template T of Level1
 */
class DeepContainer {
    /**
     * @param T $_item
     */
    public function put($_item): void {}
}

function testDeepHierarchy() {
    $c = new DeepContainer();
    $c->put(new Level1());
    $c->put(new Level2());
    $c->put(new Level3());
    $c->put(new Level4());
}

testTemplateInheritance();
testInterfaceInheritance();
testDeepHierarchy();
===expect===
