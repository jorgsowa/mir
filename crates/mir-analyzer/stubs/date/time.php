<?php

namespace Time {
    /**
     * @since 8.6
     */
    class TimeException extends \Exception {}

    /**
     * An immutable span of time with nanosecond precision.
     * @since 8.6
     */
    final readonly class Duration
    {
        public int $seconds;

        public int $nanoseconds;

        public bool $negative;

        private function __construct() {}

        public function __unserialize(array $data): void {}

        public static function fromSeconds(int $seconds, int $nanoseconds = 0): Duration {}

        public static function fromNanoseconds(int $nanoseconds): Duration {}

        public static function fromMicroseconds(int $microseconds): Duration {}

        public static function fromMilliseconds(int $milliseconds): Duration {}

        public static function fromMinutes(int $minutes): Duration {}

        public static function fromHours(int $hours): Duration {}

        /**
         * @throws TimeException
         */
        public static function fromIso8601DurationString(string $specification): Duration {}

        public function negate(): Duration {}

        public function absolute(): Duration {}

        public function add(Duration $duration): Duration {}

        public function sub(Duration $duration): Duration {}

        public function multiplyBy(int $factor): Duration {}

        public function divideBy(int $divisor): Duration {}

        public static function compare(Duration $a, Duration $b): int {}
    }
}
