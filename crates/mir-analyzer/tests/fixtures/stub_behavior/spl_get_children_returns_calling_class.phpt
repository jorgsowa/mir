===description===
`getChildren()` of `RecursiveDirectoryIterator`, `RecursiveArrayIterator`, `RecursiveRegexIterator` and `RecursiveCallbackFilterIterator` returns an instance of the calling class.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
class Directories extends RecursiveDirectoryIterator {
    public string $relative = '';

    public function getChildren(): RecursiveDirectoryIterator {
        $children = parent::getChildren();
        $children->relative = $this->relative;
        return $children;
    }
}

class Items extends RecursiveArrayIterator {
    public int $depth = 0;

    public function getChildren(): ?RecursiveArrayIterator {
        $children = parent::getChildren();
        if ($children !== null) {
            $children->depth = $this->depth + 1;
        }
        return $children;
    }
}

class Matches extends RecursiveRegexIterator {
    public int $depth = 0;

    public function getChildren(): RecursiveRegexIterator {
        $children = parent::getChildren();
        $children->depth = $this->depth + 1;
        return $children;
    }
}

class Filtered extends RecursiveCallbackFilterIterator {
    public int $depth = 0;

    public function getChildren(): RecursiveCallbackFilterIterator {
        $children = parent::getChildren();
        $children->depth = $this->depth + 1;
        return $children;
    }
}
