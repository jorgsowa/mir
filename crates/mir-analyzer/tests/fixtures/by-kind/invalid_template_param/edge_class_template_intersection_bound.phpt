===description===
Class-level template with intersection bound in namespaced file — should pass when T satisfies bound
===file===
<?php
namespace App;

interface Serializable {}
interface Loggable {}
class Entity implements Serializable, Loggable {}

/**
 * @template T of Serializable&Loggable
 */
class Repository {
    /** @param T $item */
    public function save($item): void {
        $item;
    }
}

$repo = new Repository();
$repo->save(new Entity());
===expect===
