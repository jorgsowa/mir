===description===
The loop variable is never read because the body checks an outer variable instead.
Reads of the variable in sibling loops or after the loop do not count.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Access {
    public function canView(int $user): bool { return $user > 0; }
}

class Notifier {
    public function __construct(private Access $access) {}

    /** @param list<int> $mentionedUserIds */
    public function wrong(int $currentUser, array $mentionedUserIds): void {
        foreach ($mentionedUserIds as $mentionedUserId) {
//                                    ^^^^^^^^^^^^^^^^ UnusedForeachValue: Foreach value $mentionedUserId is never read
            if (!$this->access->canView($currentUser)) {
                throw new \RuntimeException('denied');
            }
        }
    }

    /** @param list<int> $mentionedUserIds */
    public function key_only_read(int $currentUser, array $mentionedUserIds): void {
        foreach ($mentionedUserIds as $key => $mentionedUserId) {
//                                            ^^^^^^^^^^^^^^^^ UnusedForeachValue: Foreach value $mentionedUserId is never read
//                                            ^^^^^^^^^^^^^^^^ UnusedForeachValue: Foreach value $mentionedUserId is never read
            echo $key, $this->access->canView($currentUser);
        }
    }

    /** @param list<int> $mentionedUserIds */
    public function correct(array $mentionedUserIds): void {
        foreach ($mentionedUserIds as $mentionedUserId) {
            /** @mir-check $mentionedUserId is int */
            if (!$this->access->canView($mentionedUserId)) {
                throw new \RuntimeException('denied');
            }
        }
    }

    /** @param list<int> $mentionedUserIds */
    public function read_in_nested_loop(array $mentionedUserIds): void {
        foreach ($mentionedUserIds as $mentionedUserId) {
            foreach ([1, 2] as $_) {
                echo $mentionedUserId;
            }
        }
    }
}
