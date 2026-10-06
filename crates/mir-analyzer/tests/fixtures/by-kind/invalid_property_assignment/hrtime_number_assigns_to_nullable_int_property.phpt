===description===
hrtime(true) is int; the array form and a non-literal flag are still not assignable to ?int.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Timer {
    private ?int $start = null;

    public function literal(): void {
        $this->start = hrtime(true);
    }

    public function cast(bool $flag): void {
        $this->start = (int) hrtime($flag);
    }

    public function dynamic(bool $flag): void {
        $this->start = hrtime($flag);
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $start expects 'int|null', cannot assign 'int|array{0: int, 1: int}|false'
    }
}
