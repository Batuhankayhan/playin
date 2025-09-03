#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Get};
    use frame_support::weights::Weight;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type MaxNoteLen: Get<u32>;
        type MaxNotesPerAccount: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn notes_count)]
    pub type NotesCount<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NoteAdded { who: T::AccountId, total: u32 },
    }

    #[pallet::error]
    pub enum Error<T> {
        TooManyNotes,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(10_000, 0))]
        pub fn add_note(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let mut n = <NotesCount<T>>::get(&who);
            ensure!(n < T::MaxNotesPerAccount::get(), Error::<T>::TooManyNotes);
            n = n.saturating_add(1);
            <NotesCount<T>>::insert(&who, n);
            Self::deposit_event(Event::NoteAdded { who, total: n });
            Ok(())
        }
    }
}
