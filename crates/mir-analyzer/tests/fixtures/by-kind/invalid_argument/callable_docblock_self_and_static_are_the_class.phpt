===description===
`self` / `static` inside a `callable(...)` or `Closure(...)` docblock signature are the declaring class
===file===
<?php
namespace App;

final class Widget {
    /** @var array<string, \Closure(self): string> */
    private array $formatters = [];

    /** @param \Closure(self): string $f */
    public function format(\Closure $f): string {
        foreach ($this->formatters as $formatter) {
            echo $formatter($this);
        }
        return $f($this);
    }

    /** @param callable(static): int $c */
    public function each(callable $c): int {
        return $c($this);
    }

    /** @param \Closure(int): static $c */
    public function make(\Closure $c): self {
        return $c(1);
    }

    /** @param \Closure(self): string $f */
    public function wrong(\Closure $f): string {
        return $f(new \stdClass());
//                ^^^^^^^^^^^^^^^ InvalidArgument: Argument $arg0 of {closure}() expects 'self(App\Widget)', got 'stdClass'
    }
}

trait RunsCallbacks {
    /** @param \Closure(static): mixed $callback */
    public function run(\Closure $callback): mixed {
        return $callback($this);
    }
}

class Query {
    use RunsCallbacks;

    /** @param \Closure(static): mixed $callback */
    public function where(\Closure $callback): static {
        $callback($this);
        return $this;
    }

    public function orWhere(): static {
        return $this->where(fn (self $query) => $query);
    }
}

final class SqlQuery extends Query {
    public function runHere(): mixed {
        return $this->run(fn (self $query) => $query);
    }
}
