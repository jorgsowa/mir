===description===
`Alias::Case` in a docblock resolves through an aliased `use`, and a wrong enum is still rejected.
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
    enum Err { case X; }
    enum Other { case X; }
}

namespace App\Util {
    /** @template-covariant F */
    final class Res {
        /** @param \Closure(F): \Throwable $onFailure */
        public function get(\Closure $onFailure): int { return 1; }
    }
}

namespace App\UseCase {
    use App\Data\Err as E;
    use App\Util\Res;

    interface Get {
        /** @return Res<E::X> */
        public function one(): Res;
    }
}

namespace App\App {
    use App\Data\Other;
    use App\UseCase\Get;

    function t(Get $g): void {
        $g->one()->get(static fn(Other $e) => new \Exception());
//                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidArgument: Argument $onFailure of get() expects 'callable whose parameter #1 accepts App\Data\Err::X', got 'callable whose parameter #1 only accepts App\Data\Other'
        $r = $g->one();
        /** @mir-check $r is App\Util\Res<App\Data\Err::X> */
    }
}
===expect===
