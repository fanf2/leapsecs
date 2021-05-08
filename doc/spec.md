Compact formats for the leap second list
========================================

_Tony Finch_ `dot@dotat.at`

_May 2021_


abstract
--------

This memo specifies two compact formats for the leap second list: a
simple text format that uses about 3 characters per leap second; and a
very small binary format that uses about 5 bits per leap second. The
aim is to make it very cheap to distribute the full leap second list
alongside the current time.


standards and procedures
------------------------

[ITU recommendation TF.460-6][TF.460-6] Annex I specifies UTC and leap
seconds.

DTAI is the difference between TAI and UTC. A positive leap second
increases DTAI by one, and a negative leap second decreases DTAI by
one. (So far there have been no negative leap seconds.)

A leap second can occur at the end of any month (of the Gregorian
calendar, though that is not explicitly specified by [TF.460-6][]).
First preference is that leap seconds should occur at the end of June
or December. (So far there have been no leap seconds in other months.)

[TF.460-6]: http://www.itu.int/rec/R-REC-TF.460-6-200202-I


### existing formats

Leap second tables are distributed by the [IERS][] and
[NIST][NIST-IETF]. (The NIST table is made available via FTP, and
republished by the IETF and with the Olson / IANA tz database.) They
use different formats, but they contain similar information.

Each leap second is indicated by the date immediately following, i.e.
the first of the month (so far, always January and July), and the
value of DTAI starting at that date.

The tables do not explicitly say whether a leap second is positive or
negative; that is implied by the difference between successive values
of DTAI.

The first entry in the tables is for 1 Jan 1972 when DTAI became 10
seconds. That was the start of UTC rather than a leap second.

The tables also include an expiry date.

[IERS]: https://hpiers.obspm.fr/eoppc/bul/bulc/Leap_Second.dat
[NIST-IETF]: https://www.ietf.org/timezones/data/leap-seconds.list
[NIST-FTP]: ftp://time.nist.gov/pub/leap-seconds.list


### publication and expiry

The usual practice is that the IERS will issue [Bulletin C][BulC] in
January and July each year, to announce whether or not there will be a
leap second at the end of June or December (respectively).

The [IERS][] and [NIST][NIST-IETF] leap second tables are updated soon
after. January issues have expiry dates of 28th December the same
year, and July issues have an expiry date of 28th June the following
year.

So the validity period for the leap second tables is roughly 11
months.

[BulC]: https://datacenter.iers.org/availableVersions.php?id=16


compact formats
---------------

Both the compact text and binary formats are based on the same
principles.

  * The gap between leap seconds is given as a count of months.

    Months correspond exactly to the [TF.460-6][] requirements; higher
    resolution times would waste space. Relative time periods can be
    more compact than the absolute dates used by existing formats.

  * Each leap second is marked as positive or negative.

    DTAI is not given explicitly; instead, a reader can calculate it
    by accumulating the positive and negative leap seconds.

  * The expiry date is rounded down to the start of the month.

    This is so that the expiry date can be written at the end of the
    list in a similar way to the leap seconds.


text format
-----------

Here is an example of the compact text format. It list the 19 leap
seconds through the Bulletin C issued in January 1994, announcing the
leap second at the end of June 1994, and expiring in December 1994.

        6+6+12+12+12+12+12+12+12+18+12+12+24+30+24+12+18+12+12+5?

The [ABNF][] for the text format is:

            leaps       = *leap end

            leap        = gap sign

            sign        = "+" / "-"

            end         = gap "?"

            gap         = nonzero 0*2digit

            digit       = "0" / nonzero

            nonzero     = %x31-39
                            ; 1 - 9

[ABNF]: https://tools.ietf.org/html/rfc5234

The compact text leap second list is a sequence of numbers, giving the
gap between leap seconds counted in months. The numbers are separated
by leap indicators and terminated by an expiry indicator.

Each gap is a decimal number between 1 and 999 without leading zeroes.

(Fewer than three digits is not enough to represent gaps that might
occur; three digits allows for 83 years, which is expected to be more
than enough; more than three digits would cause problems for the
binary format.)

A negative leap second is indicated by a '-' and a positive leap
second by a '+'.

The last number is the number of months between the last leap second
and the expiry of the list, rounded down to a whole number of months.
The list is terminated with a '?' expiry indicator.


binary format
-------------

Here is an example of the compact binary format. It lists the 27 leap
seconds up to the time of writing in May 2021. It is shown as a hex
dump.

        00111111 12113431 2112229D 565287FA


### bytecodes

The binary format is based on bytecodes. The upper half of each
bytecode contains flags, and the lower half contains the length of a
gap.

          7   6   5   4   3       0
        +---+---+---+---+-----------+
        | W | M | N | P |  G G G G  |
        +---+---+---+---+-----------+

  * GGGG (gap) is a 4 bit number. The actual length of the gap in
    months is derived from GGGG and M as follows:

    if M == 1 then gap = (GGGG + 1)

    if M == 0 then gap = (GGGG + 1) * 6

  * NP (leap) are NTP-compatible leap indicator bits.

      * N == 0, P == 1 indicates there is a positive leap second at
        the end of the gap. (Like "+" in the text format.)

      * N == 1, P == 0 indicates there is a negaive leap second at the
        end of the gap. (Like "-" in the text format.)

      * N == 1, P == 1 indicates that it is unknown whether there is a
        leap second at the end of the gap. This represents the expiry
        time at the end of the list (like "?" in the text format).

      * N == 0, P == 0 indicates that there is no leap second at the
        end of the gap. This is used when the entire gap between leap
        seconds cannot be represented in a single bytecode.

  * M (months) indicates whether the gap is counted in units of one
    month (M == 1) or six months (M == 0).

    This uses the [TF.460-6][] preference for leap seconds at the end
    of December or June to encode the list more compactly.

  * W (wide) indicates whether the bytecode is represented in full as
    two nibbles (W == 1) or abbreviated as one nibble (W == 0).


### nibbles

The binary format is read as a sequence of 4-bit nibbles. The upper
half of each byte comes before the lower half in the sequence.

Nibbles are expanded to bytecodes as follows:

  * If the value of the next nibble is less than 8 (W is clear)

    the bits of the nibble look like 0GGG

    one nibble is consumed and expanded into 00010GGG

    that is, the value of the bytecode is the value of the nibble
    plus 0x10

    Thus a single nibble encodes the common case of a gap counted in
    units of six months with a positive leap second at the end.

  * If the value of the next nibble is 8 or more (W is set)

    the bits of the nibble look like 1MNP

    two nibbles are consumed to form the bytecode 1MNPGGGG

    Note that a wide bytecode does not have to be byte-aligned, so the
    1MNP flags can be in the lower half of one byte and the GGGG gap
    can be in the upper half of the next byte.

  * If the next nibble is the last nibble, and its value is 8 or more
    (W is set), there is no nibble to use as the lower half of the
    bytecode

    the last nibble is consumed and expanded into 1MNP0100

    This abbreviates a common case at the end of the list, in which
    WMNP == 1111 with a gap of 5 months up to the list's expiry time.


### restrictions

An NP == 11 bytecode must occur at the end of the list, and must not
occur anywhere else.

The total length of a gap between leap seconds must be no more than
999 months. The total gap comprises a sequence of NP == 00 bytecodes
terminated by an NP != 00 bytesode.

(Very long gaps are represented as a sequence of 0x8F bytecodes, WMNP
== 1000, GGGG == 15, representing a gap of 16*6 months, followed by
any remainder. A 999 month gap requires ten 0x8F bytecodes (960
months) followed by 0x82 (36 months) and 0xF2 for the last 3 months.
This O(N) encoding is not very efficient, which is why gaps are
limited to 999 months.)


### encoding gaps

The recommendations in this section should be followed by software
that generates binary leap second lists, but should not be checked by
software that reads binary leap second lists.

Gaps that are a multiple of 6 months long should be encoded as a
number of `16*6` month gaps, followed by the remainder.

Gaps up to 16 months can be encoded in one bytecode.

Other gaps should be encoded as an `X*6` month gap covering a whole
number of years, followed by a gap for the remaining few months.

If the binary list would end up as an odd number of nibbles, it can be
rounded to a whole number of bytes in two ways:

  * If the terminating bytecode is 0xF4, the final 0x4 nibble can be
    omitted.

  * Otherwise, the last single-nibble bytecode can be expanded to a
    wide bytecode by inserting a flags nibble WMNP = 1001 (0x9).


### example

In the example, most leap seconds are represented as a single nibble.
For instance, `1` represents a year between positive leap seconds.

        00111111 12113431 2112229D 565287FA

There is one long gap between leap seconds, represented as 0x9D. This
is the 7 year period between the December 1998 and December 2005 leap
seconds. WMNP == 1001, GGGG = 13, so this bytecode represents a
positive leap second at the end of a 14 * 6 month gap.

The list ends with 0x87FA. This is:

  * WMNP == 1000, GGGG = 7, representing a gap of 8*6 months with no
    leap second.

  * WMNP == 1111, GGGG = 10, representing another 11 months of gap
    after which the list expires.
