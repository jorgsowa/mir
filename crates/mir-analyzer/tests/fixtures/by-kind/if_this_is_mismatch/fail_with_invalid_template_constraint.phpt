===description===
Fail with invalid template constraint
===file===
<?php
/** @template T */
final class Option { }

/**
 * @template T
 */
final class ArrayList
{
    /**
     * @template A
     * @if-this-is ArrayList<Option<A>>
     * @return ArrayList<A>
     */
    public function compact(): ArrayList
    {
        throw new RuntimeException("???");
    }
}

/** @var ArrayList<int> $list */
$list = new ArrayList();
$numbers = $list->compact();
===expect===
