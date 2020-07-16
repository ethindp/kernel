// SPDX-License-Identifier: MPL-2.0
/// General feature set
///
/// The General feature set is the base feature set for ATA devices that conform to ATA8.
///
/// * Required commands: EXECUTE DEVICE DIAGNOSTIC, IDENTIFY DEVICE, and SET FEATURES.
/// * Optional commands: DATA SET MANAGEMENT, DATA SET MANAGEMENT XL, DOWNLOAD MICROCODE, DOWNLOAD MICROCODE DMA, FLUSH CACHE, NOP, READ BUFFER, READ BUFFER DMA, READ DMA, READ SECTOR(S), READ VERIFY SECTOR(S), SET DATE & TIME, WRITE BUFFER, WRITE BUFFER DMA, WRITE DMA, WRITE SECTOR(S), WRITE UNCORRECTABLE EXT, and ZERO EXT.
/// * Prohibited commands: DEVICE RESET, IDENTIFY PACKET DEVICE, and PACKET.
/// * Required logs: IDENTIFY DEVICE data log.
///
/// See section 4.2 of ACS-4 for information on this feature set. For information on each supported command, see sections 7.9, 7.13, 7.43, 7.5-8, 7.10, 7.17-20, 7.25, 7.29, 7.42, 7.54-56, 7.62, and 7.66-67. For information on the Identify Device Data Log, see section 9.10.
