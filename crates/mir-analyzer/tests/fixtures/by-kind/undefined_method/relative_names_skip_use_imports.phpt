===description===
`namespace\Base` names `Base` in the current namespace even when `use Other\Base;`
imports a class of the same short name: in `extends`, `implements`, trait `use`,
parameter and return types, `catch` and class constant values.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file:lib.php===
<?php
namespace Other {
    class Base { public function other(): void {} }
    interface Shape {}
    trait Greets { public function otherGreet(): void {} }
    class Failure extends \Exception {}
}

namespace App {
    class Base { public function mine(): void {} }
    interface Shape { public function area(): int; }
    trait Greets { public function greet(): string { return 'hi'; } }
    class Failure extends \Exception { public function code2(): int { return 1; } }
}
===file:use.php===
<?php
namespace App;

use Other\Base;
use Other\Shape;
use Other\Greets;
use Other\Failure;

enum Suit: string { case Hearts = 'h'; }

class Child extends namespace\Base implements namespace\Shape {
    use namespace\Greets;

    const SUIT = namespace\Suit::Hearts;

    public function area(): int { return 1; }

    public function run(namespace\Base $b): namespace\Base {
        $b->mine();
        $this->mine();
        echo $this->greet();
        return $b;
    }

    public function suit(): int {
        return self::SUIT;
    }
}

function handle(): void {
    try {
        throw new namespace\Failure();
    } catch (namespace\Failure $e) {
        echo $e->code2();
    }
}
===expect===
use.php: InvalidReturnType@26:8-26:26: Return type 'App\Suit' is not compatible with declared 'int'
