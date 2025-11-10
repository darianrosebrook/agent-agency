import { PhaseManager } from "@/components/projects/PhaseManager";
import styles from "./page.module.scss";

export default function PhasePlannerPage() {
  return (
    <div className={styles.phasePlannerPage}>
      <PhaseManager />
    </div>
  );
}
