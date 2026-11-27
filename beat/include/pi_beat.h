/* Pocket Instruments: the beat file's C ABI (docs/BEAT.md; D134, D136; issue 140).
 *
 * A beat file is snapshot version 5 (ENGINE.md section 21) written as text, one to one. Two
 * calls turn the one into the other. They are pure functions of the caller's buffers: no
 * engine, no globals, no thread rule (any thread may call them, as pi_weave), and never on the
 * audio path. Built from beat/ as a static library (libpi_beat.a, beside libpi_core.a) and as
 * a .wasm that imports nothing (web/pi_beat.wasm, beside pi_core.wasm). docs/BEAT.md is the
 * specification; where this file and it disagree, docs/BEAT.md is right.
 *
 * How a host uses it:
 *   Open a beat:  pib_read the text into a buffer of PI_SNAPSHOT_SIZE (22 100) bytes, off the
 *                 audio thread; hand the bytes, at the length it returned, to the core by the
 *                 host's ordinary load (ENGINE.md section 8: a marker in the queue, pi_load
 *                 between blocks). Tell the player about each key in `own`: "key 9 used its
 *                 writer's own sample; playing the starter pack's".
 *   Save a beat:  pib_write the latest snapshot the audio thread handed over (section 8), at
 *                 the length pi_save returned, into PIB_TEXT_MAX bytes; pass in `own` the keys
 *                 that play a sample the player loaded. A version 1 to 4 snapshot (an old
 *                 patch, an old page) is first loaded and saved by the core: pib_write takes
 *                 version 5 only.
 *
 * In WebAssembly a pointer is an offset into pi_beat.wasm's own memory (export "memory"): grow
 * it by pages (2 pages hold PIB_TEXT_MAX and a snapshot) and put the buffers there, as for
 * pi_core.wasm; copy the snapshot from one module's memory to the other's. 0 is null.
 * Exports: memory, pib_read, pib_write.
 *
 * SPDX-License-Identifier: MIT
 */
#ifndef PI_BEAT_H
#define PI_BEAT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define PIB_BEAT_VERSION 1u   /* "pocket-instruments beat 1", for snapshot version 5 */
#define PIB_TEXT_MAX 98304u   /* the most text pib_write gives for any snapshot (96 KiB): measured
                                 94 349, bounded 95 038 (beat/src/tests/size.rs). docs/BEAT.md's
                                 64 KiB was a guess, too little */
#define PIB_MSG_MAX 192u      /* the longest refusal pib_read writes, its zero byte included */
#define PIB_SNAPSHOT_MAX 22100u /* = PI_SNAPSHOT_SIZE: the most pib_read can give */

/* Results below zero. */
#define PIB_ERR_TEXT -1     /* pib_read: the text is refused; msg says where and what the line takes */
#define PIB_ERR_ROOM -2     /* the output buffer is too short (cap); nothing was written */
#define PIB_ERR_NEWER -3    /* pib_read: a newer beat: "line 1: this is beat 2; this build reads beat 1" */
#define PIB_ERR_SNAPSHOT -4 /* pib_write: not a version 5 snapshot as the core writes it */

/* Reads a beat file (UTF-8 text, `len` bytes) into snapshot version 5. Returns the snapshot's
 * length (276 to 22 100) written to `snap`, and sets *own to the keys the file's `own` line
 * names (bit k-1 for key k; those keys play the starter pack's sample). Or returns below zero
 * and writes the reason into `msg`: "line 12, column 5: ...", ended by a zero byte, cut on a
 * character boundary to msg_cap. On a refusal nothing is written to `snap` or *own. `own` and
 * `msg` may be null. */
int32_t pib_read(const uint8_t *text, uint32_t len, uint8_t *snap, uint32_t cap, uint32_t *own, uint8_t *msg,
                 uint32_t msg_cap);

/* Writes snapshot version 5 (`len` bytes; bytes after its CRC are ignored, as by pi_load) as a
 * beat file in its canonical form. `own`: the keys that play the writer's own sample (bit k-1
 * for key k); a bit for a key that does not play a sample is left out. Returns the text's
 * length, not zero-terminated, or PIB_ERR_SNAPSHOT or PIB_ERR_ROOM, writing nothing. */
int32_t pib_write(const uint8_t *snap, uint32_t len, uint32_t own, uint8_t *text, uint32_t cap);

#ifdef __cplusplus
}
#endif

#endif
