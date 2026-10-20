# Slot selection, RAUC-style. Runs on every boot via boot.scr.
test -n "${BOOT_ORDER}"  || setenv BOOT_ORDER "A B"
test -n "${BOOT_A_LEFT}" || setenv BOOT_A_LEFT 3
test -n "${BOOT_B_LEFT}" || setenv BOOT_B_LEFT 3

# Simulated slots (temporary; replaced by real kernel loads in Part B)
setenv sim_boot_A 'echo SIM: slot A booted OK; setenv BOOT_A_LEFT 3; saveenv'
setenv sim_boot_B 'echo SIM: slot B hangs; reset'

# LOADER-style commands (run these with "run <name>" at the prompt)
setenv do_boot 'echo "LOADER: booting slot ${target_slot}"; run sim_boot_${target_slot}'
setenv refresh_slots 'setenv primary_slot; setenv backup_slot; for s in "${BOOT_ORDER}"; do if test -z "${primary_slot}"; then setenv primary_slot ${s}; elif test -z "${backup_slot}"; then setenv backup_slot ${s}; fi; done'
setenv boot_image1  'setenv target_slot A; run do_boot'
setenv boot_image2  'setenv target_slot B; run do_boot'
setenv boot_primary 'run refresh_slots; setenv target_slot ${primary_slot}; run do_boot'
setenv boot_backup  'run refresh_slots; setenv target_slot ${backup_slot}; run do_boot'
setenv autoboot     'run bootcmd'

setenv slot_chosen
for BOOT_SLOT in "${BOOT_ORDER}"; do
  if test -n "${slot_chosen}"; then
    echo "skipping ${BOOT_SLOT}"
  elif test "x${BOOT_SLOT}" = "xA"; then
    if test ${BOOT_A_LEFT} -gt 0; then
      setexpr BOOT_A_LEFT ${BOOT_A_LEFT} - 1
      setenv slot_chosen A
    fi
  elif test "x${BOOT_SLOT}" = "xB"; then
    if test ${BOOT_B_LEFT} -gt 0; then
      setexpr BOOT_B_LEFT ${BOOT_B_LEFT} - 1
      setenv slot_chosen B
    fi
  fi
done

if test -n "${slot_chosen}"; then
  echo "Trying slot ${slot_chosen} (A left: ${BOOT_A_LEFT}, B left: ${BOOT_B_LEFT})"
  saveenv
  run sim_boot_${slot_chosen}
else
  echo "No slot has attempts left; resetting counters"
  setenv BOOT_A_LEFT 3
  setenv BOOT_B_LEFT 3
  saveenv
  reset
fi
