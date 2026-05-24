===description===
nested generic types with template parameters should not report false positive InvalidArgument
===file===
<?php
/** @template T */
class Container { }

/** @template K @template V */
class Map { }

/**
 * @template T
 * @param Container<Container<T>> $value
 */
function processNested(Container $value): void {}

/**
 * @template K
 * @template V
 * @param Map<K, Container<V>> $map
 */
function processMapWithContainer(Map $map): void {}

class Item { }

function test(): void {
    // Nested container with concrete type
    /** @var Container<Container<Item>> $nested */
    $nested = new Container();
    processNested($nested);

    // Map with nested container
    /** @var Map<string, Container<Item>> $mapData */
    $mapData = new Map();
    processMapWithContainer($mapData);
}
===expect===
UnusedParam@12:23: Parameter $value is never used
UnusedParam@19:33: Parameter $map is never used
