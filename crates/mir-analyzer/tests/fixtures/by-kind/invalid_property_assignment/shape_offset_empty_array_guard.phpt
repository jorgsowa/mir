===description===
Comparing a shape offset to an empty array narrows the offset's collection type
===file===
<?php
final class Bag {
    /** @var non-empty-list<int> */
    public array $items = [1];

    /**
     * @param array{items: list<int>} $a
     * @throws \Exception
     */
    public function guardThrows(array $a): void {
        if ($a['items'] === []) {
            throw new \Exception();
        }
        /** @mir-check $a['items'] is non-empty-list<int> */
        $this->items = $a['items'];
    }

    /** @param array{items: list<int>} $a */
    public function flippedOperands(array $a): void {
        if ([] === $a['items']) {
            return;
        }
        /** @mir-check $a['items'] is non-empty-list<int> */
        $this->items = $a['items'];
    }

    /** @param array{items: list<int>} $a */
    public function notIdenticalBranch(array $a): void {
        if ($a['items'] !== []) {
            /** @mir-check $a['items'] is non-empty-list<int> */
            $this->items = $a['items'];
        }
    }

    /** @param array{items: list<int>} $a */
    public function looseComparison(array $a): void {
        if ($a['items'] == []) {
            return;
        }
        /** @mir-check $a['items'] is non-empty-list<int> */
        $this->items = $a['items'];
    }

    /** @param array{outer: array{items: list<int>}} $a */
    public function nestedPath(array $a): void {
        if ($a['outer']['items'] === []) {
            return;
        }
        /** @mir-check $a['outer']['items'] is non-empty-list<int> */
        $this->items = $a['outer']['items'];
    }

    /** @param array{items: list<int>} $a */
    public function emptyBranchIsEmpty(array $a): void {
        if ($a['items'] === []) {
            /** @mir-check $a['items'] is array{} */
            $a['items'];
        }
    }

    /** @param array{items: list<int>} $a */
    public function otherKeyUntouched(array $a, array $b): void {
        if ($b['items'] === []) {
            return;
        }
        $this->items = $a['items'];
    }

    /** @param array{items: list<int>} $a */
    public function noGuardStillErrors(array $a): void {
        $this->items = $a['items'];
    }
}
===expect===
InvalidPropertyAssignment@66:8-66:34: Property $items expects 'non-empty-list<int>', cannot assign 'list<int>'
InvalidPropertyAssignment@71:8-71:34: Property $items expects 'non-empty-list<int>', cannot assign 'list<int>'
