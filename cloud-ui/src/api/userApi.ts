import type { User } from "firebase/auth";
// @ts-ignore
import { db } from "@/firebase";
import { doc, setDoc } from "firebase/firestore";

export const addUserToUsersCollection = async (user: User) => {
  const profile = {
    id: user.uid,
    name: user.displayName,
    avatar: user.photoURL,
    created: false,
  };

  try {
    await setDoc(doc(db, "users", user.uid), {
      name: user.displayName,
      avatar: user.photoURL,
    });
    profile.created = true;
  } catch (error) {
    console.error("Error adding document: ", error);
  }

  return profile;
};
