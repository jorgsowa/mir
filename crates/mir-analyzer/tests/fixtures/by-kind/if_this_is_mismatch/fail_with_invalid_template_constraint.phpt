===description===
Fail with invalid template constraint
===config===
suppress=UnusedVariable
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
IfThisIsMismatch@23:11-23:27: Cannot call ArrayList::compact() — @if-this-is requires $this to be 'ArrayList<Option<A>>', but it is 'ArrayList<int>'
