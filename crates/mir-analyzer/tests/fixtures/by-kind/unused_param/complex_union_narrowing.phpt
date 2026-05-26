===description===
complex union narrowing with multiple instanceof checks
===file===
<?php
interface Logger {
    public function log(string $msg): void;
}

interface Cache {
    public function get(string $key): mixed;
}

class Console implements Logger {
    public function log(string $msg): void {}
}

class File implements Logger {
    public function log(string $msg): void {}
}

class Memory implements Cache {
    public function get(string $key): mixed { return null; }
}

/**
 * @template T as Logger|Cache
 * @param T $handler
 */
function handleObject(Logger|Cache $handler): void {
    if ($handler instanceof Logger) {
        $handler->log("test");
    } elseif ($handler instanceof Cache) {
        $handler->get("key");
    }
}

/**
 * Three-way union narrowing
 * @template T as Console|File|Memory
 * @param T $obj
 */
function threeWayCheck(Console|File|Memory $obj): void {
    if ($obj instanceof Console) {
        $obj->log("console");
    } elseif ($obj instanceof File) {
        $obj->log("file");
    } else {
        $obj->get("memory");
    }
}
===expect===
UnusedParam@11:25: Parameter $msg is never used
UnusedParam@15:25: Parameter $msg is never used
UnusedParam@19:25: Parameter $key is never used
RedundantCondition@29:15: Condition is always true/false for type 'bool'
UndefinedMethod@45:9: Method File::get() does not exist
