===description===
`Enum::Case` in a docblock generic arg resolves the enum through `use` imports, not the current namespace.
===config===
<mir>
  <issueHandlers>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App\Data {
    enum Err { case X; case Y; }
}

namespace App\Util {
    /** @template-covariant F */
    final class Res {
        /** @param \Closure(F): \Throwable $onFailure */
        public function get(\Closure $onFailure): int { return 1; }
    }
}

namespace App\UseCase {
    use App\Data\Err;
    use App\Util\Res;

    interface Get {
        /** @return Res<Err::X> */
        public function one(): Res;

        /** @return Res<Err> */
        public function two(): Res;

        /** @return Res<\App\Data\Err::Y> */
        public function three(): Res;
    }
}

namespace App\App {
    use App\Data\Err;
    use App\UseCase\Get;

    function t(Get $g): void {
        $g->one()->get(static fn(Err $e) => new \Exception());
        $g->two()->get(static fn(Err $e) => new \Exception());
        $g->three()->get(static fn(Err $e) => new \Exception());
        $r = $g->one();
        /** @mir-check $r is App\Util\Res<App\Data\Err::X> */
    }
}
===expect===
